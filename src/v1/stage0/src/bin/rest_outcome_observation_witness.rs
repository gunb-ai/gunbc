#![allow(clippy::disallowed_macros)]

//! Executed witness for REST transport-outcome observation (DESIGN §5 fail-closed,
//! §2 one concept across realizations).
//!
//! THE DEFECT THIS PINS. The shell transport has always surfaced its own outcome —
//! `exit_success`, `exit_code`, `stderr` are declarable outputs, so a `.dag` caller can
//! branch on a failed command and return a typed refusal. The REST transport could not:
//! `dispatch_rest` turned every non-2xx into `InterpError::TypeError` and returned `Err`,
//! so no value ever reached the caller and no refusal arm could be constructed around a
//! failed request. Operations had been DECLARING error shapes the whole time —
//! `response { 401 => GitHubErrorShape, 404 => ..., 5xx => ... }` on github.Pulls.List —
//! and those arms were parsed, round-tripped by the Rust emitter to hold the self-host
//! fixed point, and never consulted by the interpreter. Declared, never realized.
//!
//! WHY IT MATTERS BEYOND TIDINESS. gunbc.roadmap_publish adjudicates whether an exact
//! commit is published, and its `PullRequestsUnreadable` arm exists precisely so that
//! "no pull request exists" and "I could not find out" stay distinguishable. With the
//! transport raising, that arm was unreachable on the live path: the only observable
//! outcomes were a successful read or a dead process, and a publication judgment that
//! cannot say "I could not find out" reports a branch unpublished when the truth is that
//! nothing was learned.
//!
//! WHAT IS CLAIMED HERE, BY EXECUTION AGAINST A REAL SOCKET. A loopback server returns
//! canned responses; the interpreter dispatches real HTTP at them through ureq. Each
//! check asserts a value the mechanism cannot produce if the change is absent or wrong:
//!
//!   1. an operation declaring `http_success`/`http_status` receives a 401 as DATA
//!   2. RED CONTROL — the same 401, an operation declaring neither, still RAISES
//!   3. a 2xx still maps its body, and the outcome fields agree with it
//!   4. a bare-array 2xx body still lands in the sole body-reading output
//!
//! Check 2 is what keeps the change from being a silent widen: observation is opt-in,
//! and every operation that did not ask for it keeps the loud failure it always had.
//! Check 4 pins the hazard the change introduces rather than removes — adding an outcome
//! field to a list operation must not strand the array, which is the shape GitHub's list
//! endpoints return.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::ExitCode;
use std::rc::Rc;
use std::thread;

use im::HashMap;

use v1_compiler::cli_run::workspace_root;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult, SourceFile};
use v1_compiler::v1_interpreter::{self, ExecutionMode, InterpError, Value};

type ModuleIndex = HashMap<String, std::path::PathBuf>;

// ---- loopback HTTP server ------------------------------------------------------

/// Serves one canned response to every connection, forever, on a background thread.
/// Bound to port 0 so concurrent runs never collide on a fixed port.
fn serve_canned(status_line: &'static str, body: &'static str) -> Result<u16, String> {
    let listener =
        TcpListener::bind("127.0.0.1:0").map_err(|e| format!("bind loopback listener: {e}"))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("read bound port: {e}"))?
        .port();
    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let _ = respond(stream, status_line, body);
        }
    });
    Ok(port)
}

fn respond(mut stream: TcpStream, status_line: &str, body: &str) -> std::io::Result<()> {
    // Drain what the client sent. The request is not inspected: these checks are about
    // how a RESPONSE is turned into a value, so varying the request would only add a way
    // for the witness to fail for a reason it does not claim.
    let mut scratch = [0u8; 8192];
    let _ = stream.read(&mut scratch)?;
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status_line,
        body.len(),
        body
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()
}

// ---- source assembly (same shape as auth_declared_but_unwired_witness) ---------

fn source_roots() -> [std::path::PathBuf; 2] {
    let ws = workspace_root();
    [ws.join("src/v1"), ws.join("dag")]
}

fn extract_module_declaration(path: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        return trimmed
            .strip_prefix("module ")
            .and_then(|rest| rest.split_whitespace().next())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
    }
    None
}

fn scan_dag_files(dir: &std::path::Path, index: &mut ModuleIndex) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dag_files(&path, index);
        } else if path.extension().map(|e| e == "dag").unwrap_or(false) {
            if let Some(module_path) = extract_module_declaration(&path) {
                index.insert(module_path, path);
            }
        }
    }
}

fn build_module_index() -> ModuleIndex {
    let mut index = HashMap::new();
    for root in source_roots() {
        if root.exists() {
            scan_dag_files(&root, &mut index);
        }
    }
    index
}

fn extract_imports(source: &str) -> Vec<String> {
    let tokens =
        v1_compiler::v1_compiler_tokenize::tokenize(source.to_string(), "test.dag".to_string());
    let source_index =
        v1_compiler::v1_std_core::build_newline_index("test.dag".to_string(), source.to_string());
    let mut source_indices = HashMap::new();
    source_indices.insert("test.dag".to_string(), source_index);
    let result = v1_compiler::v1_compiler_parse::parse(tokens, Rc::new(source_indices));
    match &result.module {
        Some(module) => v1_compiler::v1_std_core::module_imports(module.clone())
            .iter()
            .map(|imp| imp.name.clone())
            .collect(),
        None => vec![],
    }
}

fn resolve_imports_transitively(
    entry_path: &str,
    entry_content: &str,
    module_index: &ModuleIndex,
) -> Vec<Rc<SourceFile>> {
    let ws = workspace_root();
    let mut seen: HashMap<String, Rc<SourceFile>> = HashMap::new();
    let mut queue = vec![(entry_path.to_string(), entry_content.to_string())];

    while let Some((_path, content)) = queue.pop() {
        for module_path in extract_imports(&content) {
            if seen.contains_key(&module_path) {
                continue;
            }
            if let Some(file_path) = module_index.get(&module_path) {
                if let Ok(file_content) = std::fs::read_to_string(file_path) {
                    let rel_path = file_path
                        .strip_prefix(&ws)
                        .unwrap_or(file_path)
                        .to_string_lossy()
                        .to_string();
                    seen.insert(
                        module_path.clone(),
                        Rc::new(SourceFile {
                            path: rel_path.clone(),
                            content: file_content.clone(),
                        }),
                    );
                    queue.push((rel_path, file_content));
                }
            }
        }
    }

    let mut sources: Vec<Rc<SourceFile>> = seen.into_iter().map(|(_, v)| v).collect();
    sources.push(Rc::new(SourceFile {
        path: entry_path.to_string(),
        content: entry_content.to_string(),
    }));
    sources
}

fn assert_resolved_no_hard_errors(result: &ResolvedPipelineResult) -> Result<(), String> {
    let msgs: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    if msgs.is_empty() && result.graph.is_some() {
        return Ok(());
    }
    Err(format!("expected resolved graph, got diagnostics {msgs:?}"))
}

fn run_probe(
    module_index: &ModuleIndex,
    src: &str,
    entry: &str,
) -> Result<Result<Value, InterpError>, String> {
    let sources = resolve_imports_transitively("test.dag", src, module_index);
    let resolved = compile_to_resolved(Rc::new(sources.into()));
    assert_resolved_no_hard_errors(&resolved)?;
    let graph = resolved.graph.as_ref().ok_or("graph")?;
    let ctx = v1_interpreter::InterpContext::new(
        graph,
        resolved.source_indices.clone(),
        ExecutionMode::Wet,
    );
    Ok(v1_interpreter::run_in_context(&ctx, entry, false))
}

// ---- the .dag subjects ---------------------------------------------------------

fn observing_source(port: u16) -> String {
    format!(
        r#"module rest_outcome_observing

service test.Probe {{
  config {{
    endpoint: "http://127.0.0.1:{port}"
  }}
  operation Observed {{
    output {{
      ok: Bool from "http_success"
      status: Int from "http_status"
      detail: String from "message"
    }}
    transport rest {{ method: GET, path: "/thing" }}
  }}
}}

fn probe_ok() -> Bool {{
  let r = test.Probe.Observed()
  r.ok
}}

fn probe_status() -> Int {{
  let r = test.Probe.Observed()
  r.status
}}

fn probe_detail() -> String {{
  let r = test.Probe.Observed()
  r.detail
}}
"#
    )
}

fn unobserving_source(port: u16) -> String {
    format!(
        r#"module rest_outcome_unobserving

service test.Probe {{
  config {{
    endpoint: "http://127.0.0.1:{port}"
  }}
  operation Observed {{
    output {{
      detail: String from "message"
    }}
    transport rest {{ method: GET, path: "/thing" }}
  }}
}}

fn probe_detail() -> String {{
  let r = test.Probe.Observed()
  r.detail
}}
"#
    )
}

fn array_source(port: u16) -> String {
    format!(
        r#"module rest_outcome_array

service test.Probe {{
  config {{
    endpoint: "http://127.0.0.1:{port}"
  }}
  operation Listed {{
    output {{
      items: List<Int>
      ok: Bool from "http_success"
    }}
    transport rest {{ method: GET, path: "/things" }}
  }}
}}

fn probe_count() -> Int {{
  let r = test.Probe.Listed()
  count(r.items)
}}

fn probe_ok() -> Bool {{
  let r = test.Probe.Listed()
  r.ok
}}
"#
    )
}

// ---- checks --------------------------------------------------------------------

fn expect_bool(v: Result<Value, InterpError>, what: &str) -> Result<bool, String> {
    match v {
        Ok(Value::Bool(b)) => Ok(b),
        Ok(other) => Err(format!("{what}: expected Bool, got {other:?}")),
        Err(e) => Err(format!("{what}: expected a value, got error {e:?}")),
    }
}

fn expect_int(v: Result<Value, InterpError>, what: &str) -> Result<i64, String> {
    match v {
        Ok(Value::Int(i)) => Ok(i),
        Ok(other) => Err(format!("{what}: expected Int, got {other:?}")),
        Err(e) => Err(format!("{what}: expected a value, got error {e:?}")),
    }
}

fn expect_str(v: Result<Value, InterpError>, what: &str) -> Result<String, String> {
    match v {
        Ok(Value::Str(s)) => Ok(s),
        Ok(other) => Err(format!("{what}: expected Str, got {other:?}")),
        Err(e) => Err(format!("{what}: expected a value, got error {e:?}")),
    }
}

/// THE CLAIM. A declared-outcome operation receives a 401 as an ordinary record.
fn a_refused_request_arrives_as_a_value_when_declared(idx: &ModuleIndex) -> Result<(), String> {
    let port = serve_canned("401 Unauthorized", r#"{"message":"denied"}"#)?;
    let src = observing_source(port);

    let ok = expect_bool(run_probe(idx, &src, "probe_ok")?, "401 http_success")?;
    if ok {
        return Err("401 mapped http_success to true".to_string());
    }
    let status = expect_int(run_probe(idx, &src, "probe_status")?, "401 http_status")?;
    if status != 401 {
        return Err(format!("expected http_status 401, got {status}"));
    }
    // The error BODY is still mapped, so a caller can name the cause rather than
    // reporting a bare code. This is the half that makes the arm useful, not merely
    // reachable.
    let detail = expect_str(run_probe(idx, &src, "probe_detail")?, "401 body")?;
    if detail != "denied" {
        return Err(format!("expected body message 'denied', got '{detail}'"));
    }
    Ok(())
}

/// RED CONTROL. Identical response, an operation that declared no outcome output.
/// This must still raise: the change is opt-in, and an existing caller that never
/// asked to observe its status must not silently start receiving half-null records
/// where it used to fail loudly.
fn an_undeclared_operation_still_raises(idx: &ModuleIndex) -> Result<(), String> {
    let port = serve_canned("401 Unauthorized", r#"{"message":"denied"}"#)?;
    let src = unobserving_source(port);
    match run_probe(idx, &src, "probe_detail")? {
        Err(InterpError::TypeError { msg }) => {
            if !msg.contains("401") {
                return Err(format!(
                    "expected the 401 to be named in the raise, got: {msg}"
                ));
            }
            Ok(())
        }
        Err(other) => Err(format!(
            "expected a TypeError naming the status, got {other:?}"
        )),
        Ok(v) => Err(format!(
            "REGRESSION: an operation declaring no outcome output received {v:?} \
             instead of raising — observation stopped being opt-in, and every existing \
             REST caller just lost its loud failure"
        )),
    }
}

/// A 2xx still maps its body, and the outcome fields agree with it.
fn a_successful_request_still_maps_and_reports_success(idx: &ModuleIndex) -> Result<(), String> {
    let port = serve_canned("200 OK", r#"{"message":"fine"}"#)?;
    let src = observing_source(port);

    if !expect_bool(run_probe(idx, &src, "probe_ok")?, "200 http_success")? {
        return Err("200 mapped http_success to false".to_string());
    }
    let status = expect_int(run_probe(idx, &src, "probe_status")?, "200 http_status")?;
    if status != 200 {
        return Err(format!("expected http_status 200, got {status}"));
    }
    let detail = expect_str(run_probe(idx, &src, "probe_detail")?, "200 body")?;
    if detail != "fine" {
        return Err(format!("expected body message 'fine', got '{detail}'"));
    }
    Ok(())
}

/// The stranding hazard. A bare-array body must still reach the sole body-reading
/// output once an outcome field sits beside it — otherwise adding a status to a list
/// operation would quietly answer every call with an empty list, which is exactly the
/// fabricated-plausible-output shape this change exists to remove.
fn an_array_body_survives_beside_an_outcome_field(idx: &ModuleIndex) -> Result<(), String> {
    let port = serve_canned("200 OK", "[1,2,3]")?;
    let src = array_source(port);

    let n = expect_int(run_probe(idx, &src, "probe_count")?, "array body")?;
    if n != 3 {
        return Err(format!(
            "expected the 3-element array to reach `items`, got {n} — \
             the outcome field stranded the document"
        ));
    }
    if !expect_bool(run_probe(idx, &src, "probe_ok")?, "array http_success")? {
        return Err("200 array mapped http_success to false".to_string());
    }
    Ok(())
}

type WitnessCase = (&'static str, fn(&ModuleIndex) -> Result<(), String>);

const CASES: &[WitnessCase] = &[
    (
        "a_refused_request_arrives_as_a_value_when_declared",
        a_refused_request_arrives_as_a_value_when_declared,
    ),
    (
        "an_undeclared_operation_still_raises",
        an_undeclared_operation_still_raises,
    ),
    (
        "a_successful_request_still_maps_and_reports_success",
        a_successful_request_still_maps_and_reports_success,
    ),
    (
        "an_array_body_survives_beside_an_outcome_field",
        an_array_body_survives_beside_an_outcome_field,
    ),
];

fn main() -> ExitCode {
    let idx = build_module_index();
    let mut failed = 0usize;
    for (name, case) in CASES {
        match case(&idx) {
            Ok(()) => println!("PASS {name}"),
            Err(e) => {
                println!("FAIL {name}: {e}");
                failed += 1;
            }
        }
    }
    if failed == 0 {
        println!(
            "rest_outcome_observation_witness: {} case(s) green",
            CASES.len()
        );
        ExitCode::SUCCESS
    } else {
        eprintln!("rest_outcome_observation_witness: {failed} case(s) red");
        ExitCode::from(1)
    }
}
