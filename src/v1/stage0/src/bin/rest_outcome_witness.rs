#![allow(clippy::disallowed_macros)]

//! Executed .dag-side witness for `RestOutcome` in the interpreter REST kernel.

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
    serve_raw(move |mut stream| {
        let response = format!(
            "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes())?;
        stream.flush()
    })
}

fn serve_truncated_body() -> Result<u16, String> {
    serve_raw(|mut stream| {
        stream.write_all(
            b"HTTP/1.1 401 Unauthorized\r\nContent-Type: application/json\r\nContent-Length: 99\r\nConnection: close\r\n\r\nshort",
        )?;
        stream.flush()
    })
}

fn serve_raw(
    respond: impl FnOnce(TcpStream) -> std::io::Result<()> + Send + 'static,
) -> Result<u16, String> {
    let listener =
        TcpListener::bind("127.0.0.1:0").map_err(|e| format!("bind loopback listener: {e}"))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("read loopback port: {e}"))?
        .port();
    thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut request = [0u8; 8192];
            let _ = stream.read(&mut request);
            let _ = respond(stream);
        }
    });
    Ok(port)
}

fn source_roots() -> [std::path::PathBuf; 2] {
    let root = workspace_root();
    [root.join("src/v1"), root.join("dag")]
}

fn scan_dag_files(dir: &std::path::Path, index: &mut ModuleIndex) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dag_files(&path, index);
            continue;
        }
        if path.extension().is_none_or(|ext| ext != "dag") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let module = content
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .and_then(|line| line.strip_prefix("module "))
            .and_then(|rest| rest.split_whitespace().next());
        if let Some(module) = module {
            index.insert(module.to_string(), path);
        }
    }
}

fn build_module_index() -> ModuleIndex {
    let mut index = HashMap::new();
    for root in source_roots() {
        scan_dag_files(&root, &mut index);
    }
    index
}

fn extract_imports(path: &str, source: &str) -> Vec<String> {
    let tokens = v1_compiler::v1_compiler_tokenize::tokenize(source.to_string(), path.to_string());
    let mut source_indices = HashMap::new();
    source_indices.insert(
        path.to_string(),
        v1_compiler::v1_std_core::build_newline_index(path.to_string(), source.to_string()),
    );
    let parsed = v1_compiler::v1_compiler_parse::parse(tokens, Rc::new(source_indices));
    parsed
        .module
        .as_ref()
        .map(|module| {
            v1_compiler::v1_std_core::module_imports(module.clone())
                .iter()
                .map(|import| import.name.clone())
                .collect()
        })
        .unwrap_or_default()
}

fn source_closure(entry: &str, index: &ModuleIndex) -> Result<Vec<Rc<SourceFile>>, String> {
    let root = workspace_root();
    let mut sources: HashMap<String, Rc<SourceFile>> = HashMap::new();
    let mut queue = vec![("rest_outcome_probe.dag".to_string(), entry.to_string())];
    while let Some((path, content)) = queue.pop() {
        for module in extract_imports(&path, &content) {
            if sources.contains_key(&module) {
                continue;
            }
            let file = index
                .get(&module)
                .ok_or_else(|| format!("module not found while loading witness: {module}"))?;
            let imported = std::fs::read_to_string(file)
                .map_err(|e| format!("read {}: {e}", file.display()))?;
            let relative = file
                .strip_prefix(&root)
                .unwrap_or(file)
                .to_string_lossy()
                .into_owned();
            sources.insert(
                module,
                Rc::new(SourceFile {
                    path: relative.clone(),
                    content: imported.clone(),
                }),
            );
            queue.push((relative, imported));
        }
    }
    let mut result: Vec<_> = sources.into_iter().map(|(_, source)| source).collect();
    result.push(Rc::new(SourceFile {
        path: "rest_outcome_probe.dag".to_string(),
        content: entry.to_string(),
    }));
    Ok(result)
}

fn assert_resolved(result: &ResolvedPipelineResult) -> Result<(), String> {
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            v1_compiler::v1_std_core::is_interpreter_blocking_diagnostic(
                diagnostic.diagnostic.clone(),
            )
        })
        .map(|diagnostic| {
            let span = v1_compiler::v1_std_core::diagnostic_to_span(diagnostic.diagnostic.clone());
            format!(
                "{}:{}: {}",
                span.file,
                span.start,
                v1_compiler::v1_std_core::diagnostic_to_message(diagnostic.diagnostic.clone())
            )
        })
        .collect();
    if errors.is_empty() && result.graph.is_some() {
        Ok(())
    } else {
        Err(format!("witness source did not resolve: {errors:?}"))
    }
}

fn run_probe(index: &ModuleIndex, source: &str) -> Result<Result<Value, InterpError>, String> {
    let sources = source_closure(source, index)?;
    let resolved = compile_to_resolved(Rc::new(sources.into()));
    assert_resolved(&resolved)?;
    let ctx = v1_interpreter::InterpContext::new(
        resolved.graph.as_ref().ok_or("missing graph")?,
        resolved.source_indices.clone(),
        ExecutionMode::Wet,
    );
    Ok(v1_interpreter::run_in_context(&ctx, "persist", false))
}

fn outcome_source(endpoint: &str) -> String {
    format!(
        r#"module rest_outcome_probe

import extdeps.transports.rest {{ RestOutcome }}

service test.Probe {{
  config {{ endpoint: "{endpoint}" }}
  operation Observe {{
    output {{ observed: RestOutcome }}
    transport rest {{ method: GET, path: "/answer" }}
  }}
}}

fn persist() -> String {{
  let response = test.Probe.Observe()
  match response.observed {{
    RestOk => "ok"
    RestStatusRefused {{ status: s, body: b }} => concat(concat("status=", s), concat(";body=", b))
    RestTransportRefused {{ cause: c }} => concat("transport=", c)
    RestBodyUndecodable {{ status: s, cause: c }} => concat(concat("undecodable=", s), concat(";cause=", c))
  }}
}}
"#
    )
}

fn legacy_source(endpoint: &str) -> String {
    format!(
        r#"module rest_outcome_legacy_probe

service test.Probe {{
  config {{ endpoint: "{endpoint}" }}
  operation Observe {{
    output {{ body: String }}
    transport rest {{ method: GET, path: "/answer" }}
  }}
}}

fn persist() -> String {{
  let response = test.Probe.Observe()
  response.body
}}
"#
    )
}

fn expect_string(result: Result<Value, InterpError>, prefix: &str) -> Result<String, String> {
    match result {
        Ok(Value::Str(value)) if value.starts_with(prefix) => Ok(value),
        other => Err(format!(
            "expected persisted String beginning {prefix:?}, got {other:?}"
        )),
    }
}

fn status_and_body_are_persistable(index: &ModuleIndex) -> Result<(), String> {
    let port = serve_canned("401 Unauthorized", r#"{"message":"denied"}"#)?;
    let source = outcome_source(&format!("http://127.0.0.1:{port}"));
    let value = expect_string(run_probe(index, &source)?, "status=401;body=")?;
    if value.contains(r#"{"message":"denied"}"#) {
        Ok(())
    } else {
        Err(format!("persisted refusal lost its remote body: {value}"))
    }
}

fn success_projects_rest_ok(index: &ModuleIndex) -> Result<(), String> {
    let port = serve_canned("200 OK", r#"{"message":"accepted"}"#)?;
    let source = outcome_source(&format!("http://127.0.0.1:{port}"));
    expect_string(run_probe(index, &source)?, "ok").map(|_| ())
}

fn operation_without_outcome_still_raises(index: &ModuleIndex) -> Result<(), String> {
    let port = serve_canned("401 Unauthorized", r#"{"message":"denied"}"#)?;
    let source = legacy_source(&format!("http://127.0.0.1:{port}"));
    match run_probe(index, &source)? {
        Err(InterpError::TypeError { msg }) if msg == r#"HTTP 401: {"message":"denied"}"# => Ok(()),
        other => Err(format!("legacy raise behavior changed: {other:?}")),
    }
}

fn transport_failure_is_persistable(index: &ModuleIndex) -> Result<(), String> {
    let source = outcome_source("http://127.0.0.1:0");
    expect_string(run_probe(index, &source)?, "transport=").map(|_| ())
}

fn unreadable_body_is_not_fabricated_empty(index: &ModuleIndex) -> Result<(), String> {
    let port = serve_truncated_body()?;
    let source = outcome_source(&format!("http://127.0.0.1:{port}"));
    expect_string(run_probe(index, &source)?, "undecodable=401;cause=").map(|_| ())
}

type WitnessCase = (&'static str, fn(&ModuleIndex) -> Result<(), String>);

const CASES: &[WitnessCase] = &[
    (
        "status_and_body_are_persistable",
        status_and_body_are_persistable,
    ),
    ("success_projects_rest_ok", success_projects_rest_ok),
    (
        "operation_without_outcome_still_raises",
        operation_without_outcome_still_raises,
    ),
    (
        "transport_failure_is_persistable",
        transport_failure_is_persistable,
    ),
    (
        "unreadable_body_is_not_fabricated_empty",
        unreadable_body_is_not_fabricated_empty,
    ),
];

fn main() -> ExitCode {
    let index = build_module_index();
    let mut failures = 0;
    for (name, case) in CASES {
        match case(&index) {
            Ok(()) => println!("PASS {name}"),
            Err(error) => {
                failures += 1;
                eprintln!("FAIL {name}: {error}");
            }
        }
    }
    if failures == 0 {
        println!("rest_outcome_witness: {} cases green", CASES.len());
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
