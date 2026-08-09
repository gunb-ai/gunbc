//! END-TO-END RECEIPT FOR THE SERVE EVALUATION BUDGET.
//!
//! This exists because the first version of this change did not have it, and a review was right
//! to call that specification-without-execution (DESIGN §5): the budget policy had `.dag`
//! witnesses, the scope guard had unit tests, and the listener-facing behaviour this PR is
//! actually for — a runaway route returning a typed JSON refusal instead of holding the accept
//! loop forever — was proven only by hand-run curls that nothing in the tree would ever run
//! again. Hand-run receipts age into claims. This is the executing consumer.
//!
//! It drives a REAL `gunbc serve` process over a REAL socket. The three things it proves are the
//! three that unit tests structurally cannot: that the deadline is armed on the actual serve
//! path, that the refusal reaches the wire as the declared JSON, and that the listener still
//! answers afterwards — the last being the whole point, since the incident was a process that
//! stayed alive while answering nothing.
//!
//! COST: about 1.5s wall. The fixture is written to a throwaway source root containing exactly
//! one import-free module, so the graph compile that costs ~60s against `dag/` costs ~1s here.
//! That is deliberate — an end-to-end test priced at a minute would have been re-homed or
//! ignored, which is how the coverage would have been deleted a second time.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// A deliberately runaway route. It recurses through ordinary function application so the
/// cooperative stride-poll in `eval_expr` is crossed — the same shape as the sampled roadmap
/// wedge, and NOT the shape the budget cannot contain (a single blocking native primitive, which
/// never returns to `eval_expr` at all, and which this test therefore makes no claim about).
///
/// Import-free on purpose: it keeps the compiled closure at one module.
const FIXTURE: &str = r#"module serve_budget_e2e_fixture

fn spin(n: Int) -> Int {
  if n <= 0 {
    0
  } else {
    spin(n: n + 1)
  }
}

type ServeWireResponse {
  status: Int
  content_type_label: String
  body: String
}

fn serve_budget_e2e_handle(
  method: String,
  path: String,
  body: String,
  release_revision: String,
) -> ServeWireResponse {
  if path == "/spin" {
    ServeWireResponse {
      status: 200,
      content_type_label: "text/plain; charset=utf-8",
      body: concat("unreachable: ", to_string(spin(n: 1))),
    }
  } else {
    ServeWireResponse {
      status: 200,
      content_type_label: "text/plain; charset=utf-8",
      body: "healthy",
    }
  }
}
"#;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("workspace root")
        .to_path_buf()
}

struct ServeProcess {
    child: Child,
    port: u16,
}

impl Drop for ServeProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Spawns a serve process on `port` with the given budgets and waits for it to bind.
///
/// The source root must live under the workspace root — `gunbc` refuses an out-of-tree root — so
/// the fixture is written under `target/`, which is both in-tree and ignored.
fn spawn_serve(port: u16, cpu_ms: Option<u64>, wall_ms: Option<u64>) -> ServeProcess {
    let root = workspace_root();
    let src_dir = root.join("target").join(format!("serve-e2e-{port}"));
    std::fs::create_dir_all(&src_dir).expect("create fixture root");
    let entry = src_dir.join("fixture.dag");
    std::fs::write(&entry, FIXTURE).expect("write fixture");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_gunbc"));
    cmd.current_dir(&root)
        .arg("serve")
        .arg("--source-root")
        .arg(&src_dir)
        .arg("--entry")
        .arg(&entry)
        .arg("--function")
        .arg("serve_budget_e2e_handle")
        .arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(port.to_string())
        .arg("--release-revision")
        .arg("a".repeat(40));
    if let Some(ms) = cpu_ms {
        cmd.arg("--eval-budget-cpu-ms").arg(ms.to_string());
    }
    if let Some(ms) = wall_ms {
        cmd.arg("--eval-budget-wall-ms").arg(ms.to_string());
    }
    let child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn gunbc serve");

    let proc = ServeProcess { child, port };

    // Poll the socket rather than the log: binding is the fact we need, and reading it from the
    // listener itself cannot disagree with reality the way a log line can.
    let deadline = Instant::now() + Duration::from_secs(90);
    while Instant::now() < deadline {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return proc;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!("serve process never bound 127.0.0.1:{port}");
}

/// One request, one response. Deliberately hand-rolled and deliberately given a read timeout:
/// the failure this test exists for is a server that accepts and never answers, and a client
/// without a timeout would hang with it rather than reporting it.
fn request(port: u16, path: &str, timeout: Duration) -> (u16, String) {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect");
    stream
        .set_read_timeout(Some(timeout))
        .expect("read timeout");
    stream
        .set_write_timeout(Some(timeout))
        .expect("write timeout");
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"
    )
    .expect("write request");

    let mut raw = String::new();
    stream
        .read_to_string(&mut raw)
        .unwrap_or_else(|e| panic!("no response for {path} within {timeout:?}: {e}"));

    let status = raw
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or_else(|| panic!("unparseable status line in: {raw:?}"));
    let body = raw
        .split_once("\r\n\r\n")
        .map(|(_, b)| b.to_string())
        .unwrap_or_default();
    (status, body)
}

/// THE LOAD-BEARING SEQUENCE, and the order is the argument.
///
/// A refusal on its own would prove only that the budget fires. The healthy request AFTER it is
/// what proves the listener survived — the incident was a process holding its socket while
/// serving nothing, so "the runaway request ended" and "the server still works" are different
/// facts and only the second one is the deliverable. The second spin/healthy pair proves the
/// deadline is re-armed per request rather than consumed once, which is also the regression
/// control for a leaked deadline: a budget that survived its scope would refuse request 3
/// immediately against a spent baseline, and this test would see a 500 where it demands a 200.
#[test]
fn serve_refuses_runaway_route_and_keeps_serving() {
    let port = 18941;
    let serve = spawn_serve(port, Some(300), Some(5_000));

    let (status, body) = request(port, "/health", Duration::from_secs(10));
    assert_eq!(status, 200, "healthy request before the spin: {body}");
    assert!(body.contains("healthy"), "unexpected body: {body:?}");

    let started = Instant::now();
    let (status, body) = request(port, "/spin", Duration::from_secs(30));
    let elapsed = started.elapsed();
    assert_eq!(status, 500, "runaway route must refuse: {body}");
    assert!(
        body.contains("\"code\":\"evaluation_budget_exceeded\""),
        "refusal must carry the stable code: {body:?}"
    );
    assert!(
        body.contains("\"clock\":\"thread_cpu\""),
        "refusal must name the clock that fired: {body:?}"
    );
    assert!(
        body.contains("\"limit_ms\":300"),
        "refusal must report the limit it enforced: {body:?}"
    );
    assert!(
        elapsed < Duration::from_secs(20),
        "refusal must be bounded, took {elapsed:?}"
    );

    let (status, body) = request(port, "/health", Duration::from_secs(10));
    assert_eq!(
        status, 200,
        "listener must still serve after a refused route: {body}"
    );
    assert!(body.contains("healthy"), "unexpected body: {body:?}");

    // Second round: the budget re-arms per request rather than being spent.
    let (status, _) = request(port, "/spin", Duration::from_secs(30));
    assert_eq!(status, 500, "second runaway must also refuse");
    let (status, body) = request(port, "/health", Duration::from_secs(10));
    assert_eq!(status, 200, "listener must survive a second refusal");
    assert!(body.contains("healthy"), "unexpected body: {body:?}");

    drop(serve);
}

/// The wall clock must fire on its own and must SAY it was the wall clock.
///
/// Run with CPU unset, which is the configuration that caught a real defect in this change: the
/// stride poll was gated on the CPU deadline alone, so a wall-only process had no in-eval
/// crossing point at all and this route ran to a different error entirely.
///
/// Scope, stated so the green is not read as more than it is: this proves the wall ARM works and
/// reports its own clock, against a route that also burns CPU. It does NOT prove wall catches a
/// low-CPU stall — that needs an evaluation blocked inside a native primitive, which no clock
/// polled from `eval_expr` can observe.
#[test]
fn serve_wall_only_budget_fires_and_names_its_clock() {
    let port = 18942;
    let serve = spawn_serve(port, None, Some(250));

    let (status, body) = request(port, "/spin", Duration::from_secs(30));
    assert_eq!(status, 500, "wall-only budget must refuse: {body}");
    assert!(
        body.contains("\"clock\":\"monotonic_wall\""),
        "wall crossing must not report the CPU clock: {body:?}"
    );
    assert!(
        body.contains("\"limit_ms\":250"),
        "refusal must report the wall limit: {body:?}"
    );

    let (status, _) = request(port, "/health", Duration::from_secs(10));
    assert_eq!(status, 200, "listener must survive a wall refusal");

    drop(serve);
}

/// An unbounded process must behave exactly as it did before this change. This is the control
/// that keeps the default honest: both limits ship UNSET, so if arming were accidentally
/// unconditional, every deployment would start refusing and this test would see it.
#[test]
fn serve_with_no_budget_does_not_refuse_healthy_requests() {
    let port = 18943;
    let serve = spawn_serve(port, None, None);

    let (status, body) = request(port, "/health", Duration::from_secs(10));
    assert_eq!(status, 200, "unbounded process must serve normally: {body}");
    assert!(body.contains("healthy"), "unexpected body: {body:?}");

    drop(serve);
}
