//! The bound producer and caller-specified run evaluate the same fixture. Each
//! binding retains its own result and the multi-function request reports one resolve.
use std::path::Path;
use std::process::Command;
use v1_compiler::cli_run::target_invocation_host::{test_verb, Termination, TestInvocationOptions};

/// Refuse the declared phases before touching the native subject. A dormant UNIX
/// phase has a deliberately ill-typed body that a whole-reader check would reject.
fn install_refusing_native_probe(root: &Path) {
    let optional = root.join("dag/std");
    std::fs::create_dir_all(&optional).unwrap();
    std::fs::write(
        optional.join("optional.dag"),
        "module std.optional\ntype Optional<T> = Absent | Present { value: T }\n",
    )
    .unwrap();
    let gunbc = root.join("dag/gunbc");
    std::fs::create_dir_all(gunbc.join("instruments")).unwrap();
    std::fs::write(
        gunbc.join("instruments/native_serve_fixture.dag"),
        "module gunbc.instruments.native_serve_fixture\n",
    )
    .unwrap();
    std::fs::write(
        gunbc.join("instrument_targets.dag"),
        r#"module gunbc.instrument_targets
import std.optional { Optional, Present }
fn native_serve_probe_reader_for_entry(entry: String) -> Optional<String> {
  Present { value: "dag/gunbc/fixture_probe.dag" }
}
fn native_serve_probe_declared_names_for_entry(entry: String) -> List<String> { ["unjudged phase"] }
fn native_serve_probe_runs_peer(entry: String) -> Bool { false }
fn native_serve_probe_runs_refused_launch(entry: String) -> Bool { false }
fn native_serve_probe_runs_deadline_exit(entry: String) -> Bool { false }
fn native_serve_probe_runs_unix_socket(entry: String) -> Bool { false }
"#,
    )
    .unwrap();
    std::fs::write(
        gunbc.join("fixture_probe.dag"),
        r#"module gunbc.fixture_probe
fn native_serve_probe_phases_refused(declared: List<String>) -> List<String> { declared }
fn native_serve_probe_requests(peer_port: Int) -> List<String> { [] }
fn native_serve_probe_launch_argv() -> List<String> { [] }
fn native_serve_probe_standing() -> Bool { true }
fn native_serve_probe_unix_launch_argv(socket_path: String) -> Int { false }
"#,
    )
    .unwrap();
}

#[test]
fn interpreted_bindings_keep_their_result_contracts() {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("gunbc-cli-wire-{}-{stamp}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    let dag = root.join("dag/gunbc/ci");
    let workflow = root.join("src/v2/workflow");
    std::fs::create_dir_all(&dag).unwrap();
    std::fs::create_dir_all(&workflow).unwrap();
    std::fs::write(root.join(".gunbc-tree-receipt"), "fixture\n").unwrap();
    std::fs::write(
        dag.join("ci_layer_roots.dag"),
        "module gunbc.ci.ci_layer_roots\ndata witness_layer_roots: List<String> = [\"dag\", \"src/v2\"]\n",
    )
    .unwrap();
    let sweep = workflow.join("roster_import_sweep.dag");
    std::fs::write(
        &sweep,
        "module v2.workflow.roster_import_sweep\n\
         type ProcessExit = ExitSuccess | ExitFailure { code: Int, reason: String }\n\
         fn roster_import_sweep_controls_wire() -> Int { 7 / 0 }\n\
         fn roster_import_sweep_real_route_wire() -> ProcessExit { ExitSuccess }\n\
         fn unrequested_body() -> Int { false }\n",
    )
    .unwrap();
    let rehome = workflow.join("roster_rehome.dag");
    std::fs::write(
        &rehome,
        "module v2.workflow.roster_rehome\n\
         type ProcessExit = ExitSuccess | ExitFailure { code: Int, reason: String }\n\
         type CliWireResponse = CliWirePrintable { bytes: String, exit: ProcessExit } | CliWireUnprintable { cause: String }\n\
         fn rehome_controls_wire() -> CliWireResponse { CliWirePrintable { bytes: \"first reading!\", exit: ExitSuccess } }\n\
         fn second() -> CliWireResponse { CliWirePrintable { bytes: \"second reading!\", exit: ExitSuccess } }\n",
    ).unwrap();
    install_refusing_native_probe(&root);
    // This integration binary owns the process workspace. Enter the target dispatcher
    // below its host-level freshness gate, so the fixture tests the actual termination enum.
    let previous = std::env::current_dir().unwrap();
    std::env::set_var("GUNBC_WORKSPACE_ROOT", &root);
    std::env::set_current_dir(&root).unwrap();
    let instrument = |name, expected, detail: &str| {
        let label = format!("//gunbc/instruments:{name}");
        let result = test_verb(&TestInvocationOptions::single_operand(&label));
        assert_eq!(result.termination, expected, "{}", result.message);
        assert!(result.message.contains(detail), "{}", result.message);
    };
    instrument(
        "roster-import-sweep-controls",
        Termination::SubjectUnreached,
        "division by zero",
    );
    instrument(
        "roster-import-sweep-real-route",
        Termination::Refused,
        "not a CliWireResponse",
    );
    instrument(
        "roster-rehome-controls",
        Termination::ObservationHeld,
        "first reading!",
    );

    instrument(
        "native-serve",
        Termination::Refused,
        "cause=PhaseUnsatisfied",
    );
    let run = |entry: &Path, functions: &[&str], status, expected: &[&str]| {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_gunbc"));
        cmd.args(["run", "--source-root", "src/v2", "--entry"])
            .arg(entry);
        for function in functions {
            cmd.args(["--function", function]);
        }
        let output = cmd.output().unwrap();
        let log = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.status.code(), Some(status), "{log}");
        for part in expected {
            assert!(log.contains(part), "missing {part:?}: {log}");
        }
        assert_eq!(
            log.matches("[pre-entry] phase=resolve_entry_graph_inclusive ")
                .count(),
            1,
            "{log}"
        );
    };
    run(
        &sweep,
        &["roster_import_sweep_controls_wire"],
        1,
        &["DivisionByZero"],
    );
    run(&sweep, &["unrequested_body"], 1, &["resolve failed"]);
    run(
        &rehome,
        &["rehome_controls_wire", "second"],
        0,
        &["first reading!", "second reading!"],
    );
    std::env::set_current_dir(previous).unwrap();
    std::fs::remove_dir_all(root).unwrap();
}
