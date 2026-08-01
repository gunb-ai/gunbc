#![allow(clippy::disallowed_macros)]

//! Executed witness: the declared `response {}` table is the interpreter's runtime authority.
//!
//! Emit already matches status against `response_*` properties; this witness pins the
//! interpreter doing the same — decoding success arms into the operation output and
//! decoding declared error arms into typed refusals rather than a bare status string.

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
    seed_modules: &[&str],
) -> Vec<Rc<SourceFile>> {
    let ws = workspace_root();
    let mut seen: HashMap<String, Rc<SourceFile>> = HashMap::new();
    let mut queue: Vec<(String, String)> = Vec::new();

    for module_path in seed_modules {
        if seen.contains_key(*module_path) {
            continue;
        }
        if let Some(file_path) = module_index.get(*module_path) {
            if let Ok(file_content) = std::fs::read_to_string(file_path) {
                let rel_path = file_path
                    .strip_prefix(&ws)
                    .unwrap_or(file_path)
                    .to_string_lossy()
                    .to_string();
                seen.insert(
                    (*module_path).to_string(),
                    Rc::new(SourceFile {
                        path: rel_path.clone(),
                        content: file_content.clone(),
                    }),
                );
                queue.push((rel_path, file_content));
            }
        }
    }

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
        .filter(|d| {
            v1_compiler::v1_std_core::is_interpreter_blocking_diagnostic(d.diagnostic.clone())
        })
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
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
    seed_modules: &[&str],
) -> Result<Result<Value, InterpError>, String> {
    let sources = resolve_imports_transitively("test.dag", src, module_index, seed_modules);
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

fn table_source(port: u16) -> String {
    format!(
        r#"module rest_response_table

type ErrorShape {{
  message: String
}}

service test.Probe {{
  config {{
    endpoint: "http://127.0.0.1:{port}"
  }}
  operation Observed {{
    output {{
      detail: String from "message"
    }}
    transport rest {{ method: GET, path: "/thing" }}
    response {{
      200 => String
      401 => ErrorShape
    }}
  }}
}}

fn probe_detail() -> String {{
  let r = test.Probe.Observed()
  r.detail
}}
"#
    )
}

fn bare_source(port: u16) -> String {
    format!(
        r#"module rest_response_bare

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

fn expect_str(v: Result<Value, InterpError>, what: &str) -> Result<String, String> {
    match v {
        Ok(Value::Str(s)) => Ok(s),
        Ok(other) => Err(format!("{what}: expected Str, got {other:?}")),
        Err(e) => Err(format!("{what}: expected a value, got error {e:?}")),
    }
}

const REST_RESPONSE_TABLE_AUTHORITY: &[&str] = &["extdeps.transports.rest"];

fn declared_error_arm_decodes_typed_refusal(idx: &ModuleIndex) -> Result<(), String> {
    let port = serve_canned("401 Unauthorized", r#"{"message":"denied"}"#)?;
    let src = table_source(port);
    match run_probe(idx, &src, "probe_detail", REST_RESPONSE_TABLE_AUTHORITY)? {
        Err(InterpError::RestResponseRefused {
            status,
            body_type,
            detail,
            decoded,
            ..
        }) => {
            if status != 401 {
                return Err(format!("expected status 401 in refusal, got: {status}"));
            }
            if body_type != "ErrorShape" {
                return Err(format!(
                    "expected declared body type ErrorShape in refusal, got: {body_type}"
                ));
            }
            if !detail.contains("message=denied") {
                return Err(format!(
                    "expected decoded message field in refusal, got: {detail}"
                ));
            }
            if !value_record_has_str_field(&decoded, "denied") {
                return Err(format!(
                    "expected decoded ErrorShape.message=denied, got: {decoded:?}"
                ));
            }
            Ok(())
        }
        Err(other) => Err(format!("expected RestResponseRefused, got {other:?}")),
        Ok(v) => Err(format!(
            "REGRESSION: declared 401 arm returned data {v:?} instead of refusing"
        )),
    }
}

fn undeclared_status_refuses_fail_closed(idx: &ModuleIndex) -> Result<(), String> {
    let port = serve_canned("403 Forbidden", r#"{"message":"nope"}"#)?;
    let src = table_source(port);
    match run_probe(idx, &src, "probe_detail", REST_RESPONSE_TABLE_AUTHORITY)? {
        Err(InterpError::RestUndeclaredStatus { status, .. }) => {
            if status != 403 {
                return Err(format!("expected status 403 in refusal, got: {status}"));
            }
            Ok(())
        }
        Err(other) => Err(format!("expected RestUndeclaredStatus, got {other:?}")),
        Ok(v) => Err(format!(
            "REGRESSION: undeclared 403 returned data {v:?} instead of refusing"
        )),
    }
}

fn success_arm_still_maps_body(idx: &ModuleIndex) -> Result<(), String> {
    let port = serve_canned("200 OK", r#"{"message":"fine"}"#)?;
    let src = table_source(port);
    let detail = expect_str(
        run_probe(idx, &src, "probe_detail", REST_RESPONSE_TABLE_AUTHORITY)?,
        "200 body",
    )?;
    if detail != "fine" {
        return Err(format!("expected body message 'fine', got '{detail}'"));
    }
    Ok(())
}

fn no_table_still_raises_on_error(idx: &ModuleIndex) -> Result<(), String> {
    let port = serve_canned("401 Unauthorized", r#"{"message":"denied"}"#)?;
    let src = bare_source(port);
    match run_probe(idx, &src, "probe_detail", &[])? {
        Err(InterpError::TypeError { msg }) => {
            if !msg.contains("401") {
                return Err(format!("expected 401 in raise, got: {msg}"));
            }
            if msg.contains("ErrorShape") {
                return Err(format!(
                    "bare operation must not consult a response table: {msg}"
                ));
            }
            Ok(())
        }
        Err(other) => Err(format!("expected TypeError, got {other:?}")),
        Ok(v) => Err(format!(
            "REGRESSION: operation without response table received {v:?} instead of raising"
        )),
    }
}

fn response_table_without_rest_authority_refuses(idx: &ModuleIndex) -> Result<(), String> {
    let port = serve_canned("401 Unauthorized", r#"{"message":"denied"}"#)?;
    let src = table_source(port);
    match run_probe(idx, &src, "probe_detail", &[])? {
        Err(InterpError::Unimplemented { what }) => {
            if !what.contains("extdeps.transports.rest") {
                return Err(format!(
                    "expected missing-rest-authority refusal, got: {what}"
                ));
            }
            Ok(())
        }
        Err(other) => Err(format!(
            "expected Unimplemented for missing rest authority, got {other:?}"
        )),
        Ok(v) => Err(format!(
            "REGRESSION: response table without rest authority returned {v:?}"
        )),
    }
}

fn value_record_has_str_field(val: &Value, want: &str) -> bool {
    match val {
        Value::Record { fields, .. } => fields
            .iter()
            .any(|(_, v)| matches!(v, Value::Str(s) if s == want)),
        _ => false,
    }
}

fn malformed_error_body_refuses_inhabitance(idx: &ModuleIndex) -> Result<(), String> {
    let port = serve_canned("401 Unauthorized", r#"{"wrong":true}"#)?;
    let src = table_source(port);
    match run_probe(idx, &src, "probe_detail", REST_RESPONSE_TABLE_AUTHORITY)? {
        Err(InterpError::TypeError { msg }) => {
            if !msg.contains("does not inhabit ErrorShape") {
                return Err(format!("expected inhabitance refusal, got: {msg}"));
            }
            Ok(())
        }
        Err(InterpError::RestResponseRefused { .. }) => Err(
            "malformed body must not produce typed RestResponseRefused without inhabiting ErrorShape"
                .to_string(),
        ),
        Err(other) => Err(format!("expected TypeError inhabitance refusal, got {other:?}")),
        Ok(v) => Err(format!(
            "REGRESSION: malformed 401 body returned data {v:?} instead of refusing"
        )),
    }
}

fn wrong_field_type_refuses_inhabitance(idx: &ModuleIndex) -> Result<(), String> {
    let port = serve_canned("401 Unauthorized", r#"{"message":true}"#)?;
    let src = table_source(port);
    match run_probe(idx, &src, "probe_detail", REST_RESPONSE_TABLE_AUTHORITY)? {
        Err(InterpError::TypeError { msg }) => {
            if !msg.contains("does not inhabit") || !msg.contains("String") {
                return Err(format!("expected String inhabitance refusal, got: {msg}"));
            }
            Ok(())
        }
        Err(InterpError::RestResponseRefused { .. }) => Err(
            "wrong field type must not produce typed RestResponseRefused without inhabiting ErrorShape"
                .to_string(),
        ),
        Err(other) => Err(format!("expected TypeError inhabitance refusal, got {other:?}")),
        Ok(v) => Err(format!(
            "REGRESSION: wrong-type 401 body returned data {v:?} instead of refusing"
        )),
    }
}

type WitnessCase = (&'static str, fn(&ModuleIndex) -> Result<(), String>);

const CASES: &[WitnessCase] = &[
    (
        "declared_error_arm_decodes_typed_refusal",
        declared_error_arm_decodes_typed_refusal,
    ),
    (
        "response_table_without_rest_authority_refuses",
        response_table_without_rest_authority_refuses,
    ),
    (
        "wrong_field_type_refuses_inhabitance",
        wrong_field_type_refuses_inhabitance,
    ),
    (
        "malformed_error_body_refuses_inhabitance",
        malformed_error_body_refuses_inhabitance,
    ),
    (
        "undeclared_status_refuses_fail_closed",
        undeclared_status_refuses_fail_closed,
    ),
    ("success_arm_still_maps_body", success_arm_still_maps_body),
    (
        "no_table_still_raises_on_error",
        no_table_still_raises_on_error,
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
        println!("rest_response_table_witness: {} case(s) green", CASES.len());
        ExitCode::SUCCESS
    } else {
        eprintln!("rest_response_table_witness: {failed} case(s) red");
        ExitCode::from(1)
    }
}
